#pragma once

#include <QObject>
#include <QProcess>
#include <QElapsedTimer>
#include <QFutureWatcher>
#include <QPair>
#include <QVariantList>

class AudioAnalyzer final : public QObject
{
    Q_OBJECT
public:
    explicit AudioAnalyzer(QObject *parent = nullptr);
    void start();
    QVariantList levels() const { return m_levels; }

signals:
    void levelsChanged();

private:
    void consumeAudio();
    static QPair<QVariantList, QVariantList> analyze(const QByteArray &pcm,
                                                       const QVariantList &previous,
                                                       const QVariantList &previousPeaks);
    void setSilent();

    QProcess m_capture;
    QByteArray m_buffer;
    QVariantList m_levels;
    QVariantList m_bandPeaks;
    QElapsedTimer m_lastAnalysis;
    QFutureWatcher<QPair<QVariantList, QVariantList>> m_analysis;
};
