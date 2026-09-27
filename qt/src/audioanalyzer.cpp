#include "audioanalyzer.h"

#include <QStandardPaths>
#include <QtConcurrentRun>

#include <algorithm>
#include <cmath>

namespace {
constexpr int bandCount = 12;
constexpr int sampleRate = 44100;
constexpr int framesPerAnalysis = 2048;
constexpr int bytesPerFrame = 4;
constexpr double pi = 3.14159265358979323846;
}

AudioAnalyzer::AudioAnalyzer(QObject *parent) : QObject(parent)
{
    for (int index = 0; index < bandCount; ++index) {
        m_levels.append(0.0);
        m_bandPeaks.append(0.001);
    }
    connect(&m_capture, &QProcess::readyReadStandardOutput, this, &AudioAnalyzer::consumeAudio);
    connect(&m_capture, &QProcess::finished, this, &AudioAnalyzer::setSilent);
    connect(&m_analysis, &QFutureWatcherBase::finished, this, [this] {
        const auto result = m_analysis.result();
        m_levels = result.first;
        m_bandPeaks = result.second;
        emit levelsChanged();
        if (!m_buffer.isEmpty())
            consumeAudio();
    });
}

void AudioAnalyzer::start()
{
    if (m_capture.state() != QProcess::NotRunning)
        return;

    QString program = QStandardPaths::findExecutable(QStringLiteral("parec"));
    if (program.isEmpty() && !qEnvironmentVariableIsSet("FLATPAK_ID")) {
        setSilent();
        return;
    }

    QProcess sinkQuery;
    QString sinkProgram = QStringLiteral("pactl");
    QStringList sinkArguments{QStringLiteral("get-default-sink")};
    if (qEnvironmentVariableIsSet("FLATPAK_ID")) {
        sinkArguments.prepend(sinkProgram);
        sinkArguments.prepend(QStringLiteral("--host"));
        sinkProgram = QStringLiteral("flatpak-spawn");
    }
    sinkQuery.start(sinkProgram, sinkArguments);
    if (!sinkQuery.waitForFinished(2000) || sinkQuery.exitCode() != 0) {
        setSilent();
        return;
    }
    const QString monitor = QString::fromUtf8(sinkQuery.readAllStandardOutput()).trimmed()
                            + QStringLiteral(".monitor");
    if (monitor == QStringLiteral(".monitor")) {
        setSilent();
        return;
    }

    QStringList arguments{QStringLiteral("--device=%1").arg(monitor), QStringLiteral("--raw"),
                          QStringLiteral("--format=s16le"), QStringLiteral("--rate=44100"),
                          QStringLiteral("--channels=2"), QStringLiteral("--latency-msec=20"),
                          QStringLiteral("--process-time-msec=10")};
    if (qEnvironmentVariableIsSet("FLATPAK_ID")) {
        arguments.prepend(QStringLiteral("parec"));
        arguments.prepend(QStringLiteral("--host"));
        program = QStringLiteral("flatpak-spawn");
    }
    m_capture.start(program, arguments);
}

void AudioAnalyzer::consumeAudio()
{
    m_buffer += m_capture.readAllStandardOutput();
    constexpr int analysisBytes = framesPerAnalysis * bytesPerFrame;
    if (m_buffer.size() < analysisBytes)
        return;

    // Never visualize stale queued audio, and avoid flooding QML with more
    // updates than it can render smoothly.
    if (m_analysis.isRunning() || (m_lastAnalysis.isValid() && m_lastAnalysis.elapsed() < 33)) {
        if (m_buffer.size() > analysisBytes * 2)
            m_buffer = m_buffer.last(analysisBytes);
        return;
    }
    const QByteArray newest = m_buffer.last(analysisBytes);
    m_buffer.clear();
    m_lastAnalysis.restart();
    const QVariantList previous = m_levels;
    const QVariantList previousPeaks = m_bandPeaks;
    m_analysis.setFuture(QtConcurrent::run([newest, previous, previousPeaks] {
        return analyze(newest, previous, previousPeaks);
    }));
}

QPair<QVariantList, QVariantList> AudioAnalyzer::analyze(const QByteArray &pcm,
                                                         const QVariantList &previous,
                                                         const QVariantList &previousPeaks)
{
    const auto *samples = reinterpret_cast<const qint16 *>(pcm.constData());
    QVariantList next;
    next.reserve(bandCount);
    QVariantList peaks;
    peaks.reserve(bandCount);

    // Convert stereo to windowed mono once. Previously this cosine and sample
    // conversion work was repeated independently for every frequency band.
    double windowed[framesPerAnalysis];
    double signalEnergy = 0.0;
    for (int frame = 0; frame < framesPerAnalysis; ++frame) {
        const double mono = (double(samples[frame * 2]) + double(samples[frame * 2 + 1])) / 65536.0;
        signalEnergy += mono * mono;
        const double window = 0.5 - 0.5 * std::cos(2.0 * pi * frame / (framesPerAnalysis - 1));
        windowed[frame] = mono * window;
    }

    for (int band = 0; band < bandCount; ++band) {
        const double fraction = double(band) / double(bandCount - 1);
        const double frequency = 60.0 * std::pow(14000.0 / 60.0, fraction);
        const double omega = 2.0 * pi * frequency / sampleRate;
        const double coefficient = 2.0 * std::cos(omega);
        double q0 = 0.0, q1 = 0.0, q2 = 0.0;
        for (int frame = 0; frame < framesPerAnalysis; ++frame) {
            q0 = coefficient * q1 - q2 + windowed[frame];
            q2 = q1;
            q1 = q0;
        }
        const double power = std::sqrt(std::max(0.0, q1 * q1 + q2 * q2 - coefficient * q1 * q2))
                             / framesPerAnalysis;
        next.append(power);
    }

    const bool silent = std::sqrt(signalEnergy / framesPerAnalysis) < 0.0006;
    for (int band = 0; band < bandCount; ++band) {
        const double power = next[band].toDouble();
        const double peak = std::max(0.00001, std::max(power, previousPeaks[band].toDouble() * 0.975));
        peaks.append(peak);
        const double normalized = silent ? 0.0
            : std::pow(std::clamp(power / peak, 0.0, 1.0), 0.72);
        next[band] = previous[band].toDouble() * 0.45 + normalized * 0.55;
    }
    return {next, peaks};
}

void AudioAnalyzer::setSilent()
{
    for (int index = 0; index < m_levels.size(); ++index)
        m_levels[index] = 0.0;
    emit levelsChanged();
}
