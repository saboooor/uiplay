#include "appcontroller.h"
#include "metadatareader.h"
#include "mprisservice.h"
#include "discordrpc.h"

#include <QDBusConnection>
#include <QDBusMessage>
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QCryptographicHash>
#include <QJsonDocument>
#include <QJsonObject>
#include <QNetworkReply>
#include <QNetworkRequest>
#include <QNetworkProxy>
#include <QProcessEnvironment>
#include <QRegularExpression>
#include <QStandardPaths>
#include <QTimer>
#include <QUrlQuery>

#include <sys/stat.h>

AppController::AppController(QObject *parent) : QObject(parent)
{
    m_dacpNetwork.setProxy(QNetworkProxy::NoProxy);
    QDir().mkpath(configPath());
    QFile settings(configPath(QStringLiteral("settings.json")));
    if (settings.open(QIODevice::ReadOnly)) {
        const QJsonObject object = QJsonDocument::fromJson(settings.readAll()).object();
        m_receiverName = object.value(QStringLiteral("Name")).toString(m_receiverName);
        m_provider = object.value(QStringLiteral("Provider")).toString(m_provider);
    }

    connect(&m_receiver, &QProcess::readyReadStandardOutput, this, &AppController::readStandardOutput);
    connect(&m_receiver, &QProcess::readyReadStandardError, this, &AppController::readStandardError);
    connect(&m_receiver, &QProcess::finished, this, &AppController::receiverFinished);
    m_mpris = new MprisService(this, this);
    m_discord = new DiscordRpc(this, this);
}

AppController::~AppController()
{
    stopReceiver();
}

bool AppController::running() const
{
    return m_receiver.state() != QProcess::NotRunning;
}

void AppController::setReceiverName(const QString &value)
{
    if (m_receiverName == value)
        return;
    m_receiverName = value;
    emit settingsChanged();
}

void AppController::setProvider(const QString &value)
{
    if (m_provider == value)
        return;
    m_provider = value;
    emit settingsChanged();
}

void AppController::saveSettings()
{
    QFile file(configPath(QStringLiteral("settings.json")));
    if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
        appendLog(QStringLiteral("Failed to save settings: %1").arg(file.errorString()));
        return;
    }
    file.write(QJsonDocument(QJsonObject{{QStringLiteral("Name"), m_receiverName},
                                         {QStringLiteral("Provider"), m_provider}}).toJson());
}

QString AppController::configPath(const QString &file) const
{
    const QString directory = QStandardPaths::writableLocation(QStandardPaths::ConfigLocation)
                              + QStringLiteral("/uiplay");
    return file.isEmpty() ? directory : directory + QLatin1Char('/') + file;
}

bool AppController::executableAvailable(const QString &name) const
{
    if (qEnvironmentVariableIsSet("FLATPAK_ID")) {
        QProcess probe;
        probe.start(QStringLiteral("flatpak-spawn"),
                    {QStringLiteral("--host"), QStringLiteral("which"), name});
        return probe.waitForFinished(3000) && probe.exitStatus() == QProcess::NormalExit
               && probe.exitCode() == 0;
    }
    return !QStandardPaths::findExecutable(name).isEmpty();
}

void AppController::startReceiver()
{
    if (running())
        return;

    resetSession();
    QString selected = m_provider;
    if (selected == QStringLiteral("shairport") && !executableAvailable(QStringLiteral("shairport-sync")))
        selected = executableAvailable(QStringLiteral("uxplay")) ? QStringLiteral("uxplay") : QString();
    if (selected == QStringLiteral("uxplay") && !executableAvailable(QStringLiteral("uxplay")))
        selected = executableAvailable(QStringLiteral("shairport-sync")) ? QStringLiteral("shairport") : QString();
    if (selected.isEmpty()) {
        appendLog(QStringLiteral("Neither Shairport Sync nor UxPlay is installed."));
        return;
    }
    if (selected != m_provider)
        appendLog(QStringLiteral("Selected provider is unavailable; using %1.").arg(selected));

    if (selected == QStringLiteral("shairport")) {
        if (!prepareMetadataPipe())
            return;
        const QString config = configPath(QStringLiteral("shairport-sync.conf"));
        QFile file(config);
        if (!file.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
            appendLog(QStringLiteral("Failed to create Shairport configuration."));
            return;
        }
        file.write("general = {\n  dbus_service_bus = \"session\";\n  mpris_service_bus = \"session\";\n};\n");
        file.close();

        const QString pipe = configPath(QStringLiteral("shairport-metadata"));
        m_metadataReader = new MetadataReader(pipe, this);
        connect(m_metadataReader, &MetadataReader::item, this, &AppController::handleMetadata);
        connect(m_metadataReader, &MetadataReader::error, this, &AppController::appendLog);
        m_metadataReader->start();
        m_receiver.setProgram(QStringLiteral("shairport-sync"));
        m_receiver.setArguments({QStringLiteral("-c"), config, QStringLiteral("-a"), m_receiverName,
                                 QStringLiteral("-M"), QStringLiteral("--metadata-pipename=%1").arg(pipe),
                                 QStringLiteral("-g")});
    } else {
        QFile::remove(configPath(QStringLiteral("uxplay.dacp")));
        m_receiver.setProgram(QStringLiteral("stdbuf"));
        m_receiver.setArguments({QStringLiteral("-oL"), QStringLiteral("uxplay"), QStringLiteral("-n"),
                                 m_receiverName, QStringLiteral("-ca"), configPath(QStringLiteral("albumart.png")),
                                 QStringLiteral("-dacp"), configPath(QStringLiteral("uxplay.dacp")),
                                 QStringLiteral("-async")});
        sanitizeAppImageEnvironment();
    }

    if (qEnvironmentVariableIsSet("FLATPAK_ID")) {
        QStringList hostArguments{QStringLiteral("--host"), QStringLiteral("--watch-bus"),
                                  m_receiver.program()};
        hostArguments.append(m_receiver.arguments());
        m_receiver.setProgram(QStringLiteral("flatpak-spawn"));
        m_receiver.setArguments(hostArguments);
    }

    appendLog(QStringLiteral("Starting %1 receiver…").arg(selected));
    m_receiver.start();
    if (!m_receiver.waitForStarted(3000)) {
        appendLog(QStringLiteral("Failed to start receiver: %1").arg(m_receiver.errorString()));
        if (m_metadataReader) {
            m_metadataReader->stop();
            m_metadataReader->deleteLater();
            m_metadataReader = nullptr;
        }
    }
    emit runningChanged();
}

void AppController::stopReceiver()
{
    if (m_metadataReader) {
        m_metadataReader->stop();
        delete m_metadataReader;
        m_metadataReader = nullptr;
    }
    if (!running())
        return;
    m_receiver.terminate();
    if (!m_receiver.waitForFinished(2000)) {
        m_receiver.kill();
        m_receiver.waitForFinished(1000);
    }
    emit runningChanged();
}

void AppController::restartReceiver()
{
    saveSettings();
    stopReceiver();
    startReceiver();
}

bool AppController::prepareMetadataPipe()
{
    const QByteArray path = QFile::encodeName(configPath(QStringLiteral("shairport-metadata")));
    struct stat info {};
    if (::stat(path.constData(), &info) == 0) {
        if (S_ISFIFO(info.st_mode))
            return true;
        if (!QFile::remove(QString::fromLocal8Bit(path))) {
            appendLog(QStringLiteral("Could not replace existing metadata path."));
            return false;
        }
    }
    if (::mkfifo(path.constData(), 0600) != 0) {
        appendLog(QStringLiteral("Failed to create Shairport metadata pipe."));
        return false;
    }
    return true;
}

void AppController::readStandardOutput()
{
    m_stdoutBuffer += m_receiver.readAllStandardOutput();
    m_stdoutBuffer.replace('\r', '\n');
    qsizetype newline = 0;
    while ((newline = m_stdoutBuffer.indexOf('\n')) >= 0) {
        const QString line = QString::fromUtf8(m_stdoutBuffer.first(newline)).trimmed();
        m_stdoutBuffer.remove(0, newline + 1);
        if (!line.isEmpty()) {
            appendLog(line);
            parseReceiverLine(line);
        }
    }
}

void AppController::readStandardError()
{
    const QStringList lines = QString::fromUtf8(m_receiver.readAllStandardError()).split(QLatin1Char('\n'));
    for (const QString &line : lines)
        if (!line.trimmed().isEmpty())
            appendLog(QStringLiteral("[STDERR] %1").arg(line.trimmed()));
}

void AppController::receiverFinished(int exitCode, QProcess::ExitStatus status)
{
    appendLog(QStringLiteral("Receiver stopped (exit %1%2).").arg(exitCode).arg(
        status == QProcess::CrashExit ? QStringLiteral(", crashed") : QString()));
    emit runningChanged();
}

void AppController::appendLog(const QString &line)
{
    if (line.isEmpty())
        return;
    m_logs += line + QLatin1Char('\n');
    constexpr qsizetype maximum = 100000;
    if (m_logs.size() > maximum)
        m_logs.remove(0, m_logs.size() - maximum);
    emit logsChanged();
}

void AppController::parseReceiverLine(const QString &line)
{
    auto capture = [&line](const char *pattern) {
        return QRegularExpression(QString::fromLatin1(pattern)).match(line);
    };
    QRegularExpressionMatch match;
    bool metadataChanged = false;
    if ((match = capture("^Title: (.*)$")).hasMatch()) { m_title = match.captured(1); metadataChanged = true; }
    else if ((match = capture("^Artist: (.*)$")).hasMatch()) { m_artist = match.captured(1); metadataChanged = true; }
    else if ((match = capture("^Album: (.*)$")).hasMatch()) { m_album = match.captured(1); metadataChanged = true; }
    else if ((match = capture("^Genre: (.*)$")).hasMatch()) { m_genre = match.captured(1); metadataChanged = true; }
    else if ((match = capture("audio progress \\(min:sec\\):\\s*(\\d+:\\d+);\\s*remaining:\\s*(\\d+:\\d+);\\s*track length\\s*(\\d+:\\d+)")).hasMatch()) {
        m_progressSeconds = parseTime(match.captured(1));
        m_lengthSeconds = parseTime(match.captured(3));
        emit progressChanged();
    } else if ((match = capture("connection request from (.*) with deviceID = (.*)")).hasMatch()) {
        m_currentDeviceId = match.captured(2);
        updateDevice(m_currentDeviceId, match.captured(1));
    } else if ((match = capture("Accepted (.*) client on socket (.*)")).hasMatch()) {
        m_socket = match.captured(2);
    } else if ((match = capture("Client identified as User-Agent: (.*)")).hasMatch()) {
        for (QVariant &entry : m_devices) {
            QVariantMap device = entry.toMap();
            if (device.value(QStringLiteral("deviceId")).toString() == m_currentDeviceId) {
                device[QStringLiteral("userAgent")] = match.captured(1);
                device[QStringLiteral("connected")] = true;
                entry = device;
            }
        }
        emit devicesChanged();
    } else if ((match = capture("start audio connection, format (.*)")).hasMatch()) {
        for (QVariant &entry : m_devices) {
            QVariantMap device = entry.toMap();
            if (device.value(QStringLiteral("deviceId")).toString() == m_currentDeviceId) {
                device[QStringLiteral("format")] = match.captured(1);
                entry = device;
            }
        }
        emit devicesChanged();
    } else if ((match = capture("Connection closed for socket (.*)")).hasMatch()) {
        for (QVariant &entry : m_devices) {
            QVariantMap device = entry.toMap();
            if (device.value(QStringLiteral("socket")).toString() == match.captured(1)) {
                device[QStringLiteral("connected")] = false;
                entry = device;
            }
        }
        emit devicesChanged();
    } else if (line.contains(QStringLiteral("coverart size"))) {
        loadAlbumArt();
    }
    if (metadataChanged)
        emit nowPlayingChanged();
}

void AppController::handleMetadata(const QString &kind, const QString &code, const QByteArray &data)
{
    const QString value = QString::fromUtf8(data);
    QString line;
    if (kind == QStringLiteral("core") && code == QStringLiteral("minm")) line = QStringLiteral("Title: %1").arg(value);
    else if (kind == QStringLiteral("core") && code == QStringLiteral("asar")) line = QStringLiteral("Artist: %1").arg(value);
    else if (kind == QStringLiteral("core") && code == QStringLiteral("asal")) line = QStringLiteral("Album: %1").arg(value);
    else if (kind == QStringLiteral("core") && code == QStringLiteral("asgn")) line = QStringLiteral("Genre: %1").arg(value);
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("cdid")) m_currentDeviceId = value;
    else if (kind == QStringLiteral("ssnc") && (code == QStringLiteral("clip") || code == QStringLiteral("conn"))) m_clientIp = value;
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("dapo")) m_dacpPort = value.toUShort();
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("acre")) m_activeRemote = value;
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("snam")) { updateDevice(m_currentDeviceId, value); return; }
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("snua")) line = QStringLiteral("Client identified as User-Agent: %1").arg(value);
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("pbeg")) line = QStringLiteral("Accepted Shairport client on socket shairport");
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("pend")) line = QStringLiteral("Connection closed for socket shairport");
    else if (code == QStringLiteral("asfm")) line = QStringLiteral("start audio connection, format %1").arg(value);
    else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("prgr")) {
        const QStringList frames = value.split(QLatin1Char('/'));
        if (frames.size() == 3) {
            const quint32 start = frames[0].toUInt(), current = frames[1].toUInt(), end = frames[2].toUInt();
            const quint32 position = current - start, length = end - start;
            if (position <= length)
                line = QStringLiteral("audio progress (min:sec): %1; remaining: %2; track length %3")
                           .arg(formatTime(position / 44100), formatTime((length - position) / 44100), formatTime(length / 44100));
        }
    } else if (kind == QStringLiteral("ssnc") && code == QStringLiteral("PICT")) {
        QFile art(configPath(QStringLiteral("albumart.png")));
        if (art.open(QIODevice::WriteOnly | QIODevice::Truncate)) {
            art.write(data);
            art.close();
            line = QStringLiteral("coverart size %1").arg(data.size());
        }
    }
    if (!line.isEmpty()) {
        appendLog(line);
        parseReceiverLine(line);
    }
}

void AppController::updateDevice(const QString &id, const QString &name)
{
    if (id.isEmpty())
        return;
    for (QVariant &entry : m_devices) {
        QVariantMap device = entry.toMap();
        if (device.value(QStringLiteral("deviceId")).toString() == id) {
            device[QStringLiteral("name")] = name;
            device[QStringLiteral("socket")] = m_socket;
            entry = device;
            emit devicesChanged();
            return;
        }
    }
    m_devices.append(QVariantMap{{QStringLiteral("deviceId"), id}, {QStringLiteral("name"), name},
                                 {QStringLiteral("socket"), m_socket}, {QStringLiteral("connected"), false}});
    emit devicesChanged();
}

void AppController::forgetDevice(const QString &deviceId)
{
    for (qsizetype index = m_devices.size() - 1; index >= 0; --index)
        if (m_devices[index].toMap().value(QStringLiteral("deviceId")).toString() == deviceId)
            m_devices.removeAt(index);
    emit devicesChanged();
}

void AppController::loadAlbumArt()
{
    const QString path = configPath(QStringLiteral("albumart.png"));
    QFile file(path);
    if (!file.open(QIODevice::ReadOnly))
        return;
    const QByteArray data = file.readAll();
    m_albumArt = QUrl::fromLocalFile(path).toString() + QStringLiteral("?v=%1").arg(++m_artRevision);
    emit nowPlayingChanged();
    uploadAlbumArt(data);
}

void AppController::resetSession()
{
    m_title.clear(); m_artist.clear(); m_album.clear(); m_genre.clear(); m_albumArt.clear();
    m_remoteAlbumArt.clear();
    m_progressSeconds = 0; m_lengthSeconds = 0; m_currentDeviceId.clear(); m_socket.clear();
    m_clientIp.clear(); m_dacpPort = 0; m_activeRemote.clear();
    m_playing = true;
    emit nowPlayingChanged();
    emit progressChanged();
}

int AppController::parseTime(const QString &value)
{
    const QStringList parts = value.split(QLatin1Char(':'));
    return parts.size() == 2 ? parts[0].toInt() * 60 + parts[1].toInt() : 0;
}

QString AppController::formatTime(int seconds)
{
    return QStringLiteral("%1:%2").arg(seconds / 60).arg(seconds % 60, 2, 10, QLatin1Char('0'));
}

QString AppController::progressText() const { return formatTime(m_progressSeconds); }
QString AppController::lengthText() const { return formatTime(m_lengthSeconds); }

void AppController::playbackControl(const QString &control)
{
    const QString method = control == QStringLiteral("previous") ? QStringLiteral("Previous")
                           : control == QStringLiteral("next") ? QStringLiteral("Next")
                                                               : QStringLiteral("PlayPause");
    const QString dacpCommand = control == QStringLiteral("previous") ? QStringLiteral("previtem")
                                : control == QStringLiteral("next") ? QStringLiteral("nextitem")
                                                                    : QStringLiteral("playpause");
    if (sendDacpControl(dacpCommand))
        return;
    sendDbusControl(method);
}

void AppController::sendDbusControl(const QString &method)
{
    QDBusMessage message = QDBusMessage::createMethodCall(
        QStringLiteral("org.mpris.MediaPlayer2.ShairportSync"), QStringLiteral("/org/mpris/MediaPlayer2"),
        QStringLiteral("org.mpris.MediaPlayer2.Player"), method);
    QDBusMessage reply = QDBusConnection::sessionBus().call(message, QDBus::Block, 2000);
    if (reply.type() == QDBusMessage::ErrorMessage) {
        message = QDBusMessage::createMethodCall(QStringLiteral("org.gnome.ShairportSync"),
            QStringLiteral("/org/gnome/ShairportSync"), QStringLiteral("org.gnome.ShairportSync.RemoteControl"), method);
        reply = QDBusConnection::sessionBus().call(message, QDBus::Block, 2000);
    }
    if (reply.type() == QDBusMessage::ErrorMessage)
        appendLog(QStringLiteral("Playback control failed: %1").arg(reply.errorMessage()));
    else if (method == QStringLiteral("PlayPause")) {
        m_playing = !m_playing;
        emit nowPlayingChanged();
    }
}

bool AppController::sendDacpControl(const QString &command)
{
    QString host = m_clientIp;
    quint16 port = m_dacpPort;
    QString remote = m_activeRemote;

    if (host.isEmpty() || port == 0 || remote.isEmpty()) {
        QFile credentials(configPath(QStringLiteral("uxplay.dacp")));
        if (!credentials.open(QIODevice::ReadOnly))
            return false;
        QString dacpId;
        const QStringList lines = QString::fromUtf8(credentials.readAll()).split(QLatin1Char('\n'), Qt::SkipEmptyParts);
        for (const QString &raw : lines) {
            const QString line = raw.trimmed();
            const qsizetype separator = std::max(line.indexOf(QLatin1Char(':')), line.indexOf(QLatin1Char('=')));
            if (separator > 0) {
                const QString key = line.first(separator).trimmed().toLower();
                const QString value = line.sliced(separator + 1).trimmed();
                if (key == QStringLiteral("dacp-id")) dacpId = value;
                else if (key == QStringLiteral("active-remote")) remote = value;
            }
        }
        if (dacpId.isEmpty() && lines.size() >= 2) {
            dacpId = lines[0].trimmed();
            remote = lines[1].trimmed();
        }
        if (dacpId.isEmpty() || remote.isEmpty())
            return false;

        QProcess resolver;
        QString resolverProgram = QStringLiteral("avahi-browse");
        QStringList resolverArguments{QStringLiteral("--resolve"), QStringLiteral("--cache"),
            QStringLiteral("--parsable"), QStringLiteral("--no-db-lookup"), QStringLiteral("_dacp._tcp")};
        if (qEnvironmentVariableIsSet("FLATPAK_ID")) {
            resolverArguments.prepend(resolverProgram);
            resolverArguments.prepend(QStringLiteral("--host"));
            resolverProgram = QStringLiteral("flatpak-spawn");
        }
        resolver.start(resolverProgram, resolverArguments);
        if (!resolver.waitForFinished(3000)) {
            resolver.kill();
            return false;
        }
        const QString service = QStringLiteral("iTunes_Ctrl_%1").arg(dacpId);
        const QStringList output = QString::fromUtf8(resolver.readAllStandardOutput()).split(QLatin1Char('\n'));
        for (const QString &line : output) {
            const QStringList fields = line.split(QLatin1Char(';'));
            if (fields.size() > 8 && fields[0] == QStringLiteral("=") && fields[3] == service) {
                host = fields[7];
                port = fields[8].toUShort();
                break;
            }
        }
    }
    if (host.isEmpty() || port == 0 || remote.isEmpty())
        return false;

    QUrl url;
    url.setScheme(QStringLiteral("http"));
    url.setHost(host);
    url.setPort(port);
    url.setPath(QStringLiteral("/ctrl-int/1/") + command);
    QNetworkRequest request(url);
    request.setTransferTimeout(5000);
    request.setRawHeader("Active-Remote", remote.toUtf8());
    request.setRawHeader("Connection", "close");
    QNetworkReply *reply = m_dacpNetwork.get(request);
    connect(reply, &QNetworkReply::finished, this, [this, reply, command] {
        if (reply->error() != QNetworkReply::NoError) {
            appendLog(QStringLiteral("DACP playback control failed: %1").arg(reply->errorString()));
            const QString method = command == QStringLiteral("previtem") ? QStringLiteral("Previous")
                                   : command == QStringLiteral("nextitem") ? QStringLiteral("Next")
                                                                            : QStringLiteral("PlayPause");
            sendDbusControl(method);
        } else if (command == QStringLiteral("playpause")) {
            m_playing = !m_playing;
            emit nowPlayingChanged();
        }
        reply->deleteLater();
    });
    return true;
}

QString AppController::appSecret() const
{
    const QString environment = qEnvironmentVariable("UIPLAY_APP_SECRET");
    if (!environment.isEmpty())
        return environment;
    const QStringList candidates{
        QCoreApplication::applicationDirPath() + QStringLiteral("/../../src-tauri/.env"),
        QDir::currentPath() + QStringLiteral("/src-tauri/.env")};
    for (const QString &path : candidates) {
        QFile file(path);
        if (!file.open(QIODevice::ReadOnly))
            continue;
        for (const QByteArray &line : file.readAll().split('\n'))
            if (line.startsWith("APP_SECRET="))
                return QString::fromUtf8(line.sliced(sizeof("APP_SECRET=") - 1)).trimmed();
    }
    return {};
}

void AppController::uploadAlbumArt(const QByteArray &data)
{
    const QByteArray hash = QCryptographicHash::hash(data, QCryptographicHash::Sha256);
    if (hash == m_lastArtHash || m_currentDeviceId.isEmpty())
        return;
    const QString secret = appSecret();
    if (secret.isEmpty()) {
        appendLog(QStringLiteral("Album-art upload disabled: set UIPLAY_APP_SECRET."));
        return;
    }
    auto upload = [this, data, hash] {
        QUrl url(QStringLiteral("https://uiplay.luminescent.dev/upload"));
        QUrlQuery query;
        query.addQueryItem(QStringLiteral("device_id"), m_currentDeviceId);
        url.setQuery(query);
        QNetworkRequest request(url);
        request.setRawHeader("Authorization", QByteArrayLiteral("Bearer ") + m_authToken.toUtf8());
        request.setHeader(QNetworkRequest::ContentTypeHeader, QStringLiteral("image/png"));
        QNetworkReply *reply = m_network.post(request, data);
        connect(reply, &QNetworkReply::finished, this, [this, reply, hash] {
            if (reply->error() == QNetworkReply::NoError) {
                m_remoteAlbumArt = QString::fromUtf8(reply->readAll()).trimmed();
                m_lastArtHash = hash;
                appendLog(QStringLiteral("Uploaded album art to CDN."));
                emit nowPlayingChanged();
            } else {
                appendLog(QStringLiteral("Album-art upload failed: %1").arg(reply->errorString()));
            }
            reply->deleteLater();
        });
    };
    if (!m_authToken.isEmpty()) {
        upload();
        return;
    }
    QUrl url(QStringLiteral("https://uiplay.luminescent.dev/auth"));
    QUrlQuery query;
    query.addQueryItem(QStringLiteral("device_id"), m_currentDeviceId);
    url.setQuery(query);
    QNetworkRequest request(url);
    request.setRawHeader("X-App-Secret", secret.toUtf8());
    QNetworkReply *reply = m_network.post(request, QByteArray{});
    connect(reply, &QNetworkReply::finished, this, [this, reply, upload] {
        if (reply->error() == QNetworkReply::NoError) {
            m_authToken = QString::fromUtf8(reply->readAll()).trimmed();
            upload();
        } else {
            appendLog(QStringLiteral("Album-art authentication failed: %1").arg(reply->errorString()));
        }
        reply->deleteLater();
    });
}

void AppController::sanitizeAppImageEnvironment()
{
    const QString appDir = qEnvironmentVariable("APPDIR");
    if (appDir.isEmpty())
        return;
    QProcessEnvironment environment = QProcessEnvironment::systemEnvironment();
    const QStringList pathVariables{QStringLiteral("PATH"), QStringLiteral("LD_LIBRARY_PATH"),
        QStringLiteral("GST_PLUGIN_PATH"), QStringLiteral("GST_PLUGIN_PATH_1_0"),
        QStringLiteral("GST_PLUGIN_SYSTEM_PATH"), QStringLiteral("GST_PLUGIN_SYSTEM_PATH_1_0")};
    for (const QString &name : pathVariables) {
        QStringList clean;
        for (const QString &path : environment.value(name).split(QLatin1Char(':'), Qt::SkipEmptyParts))
            if (!QDir::cleanPath(path).startsWith(QDir::cleanPath(appDir)))
                clean.append(path);
        if (clean.isEmpty()) environment.remove(name); else environment.insert(name, clean.join(QLatin1Char(':')));
    }
    for (const QString &name : {QStringLiteral("GST_PLUGIN_SCANNER"), QStringLiteral("GST_PLUGIN_SCANNER_1_0")})
        if (environment.value(name).startsWith(appDir))
            environment.remove(name);
    m_receiver.setProcessEnvironment(environment);
}
