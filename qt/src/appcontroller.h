#pragma once

#include <QNetworkAccessManager>
#include <QObject>
#include <QProcess>
#include <QVariantList>

class MetadataReader;
class MprisService;
class DiscordRpc;

class AppController final : public QObject
{
    Q_OBJECT
    Q_PROPERTY(QString receiverName READ receiverName WRITE setReceiverName NOTIFY settingsChanged)
    Q_PROPERTY(QString provider READ provider WRITE setProvider NOTIFY settingsChanged)
    Q_PROPERTY(QString title READ title NOTIFY nowPlayingChanged)
    Q_PROPERTY(QString artist READ artist NOTIFY nowPlayingChanged)
    Q_PROPERTY(QString album READ album NOTIFY nowPlayingChanged)
    Q_PROPERTY(QString genre READ genre NOTIFY nowPlayingChanged)
    Q_PROPERTY(QString albumArt READ albumArt NOTIFY nowPlayingChanged)
    Q_PROPERTY(int progressSeconds READ progressSeconds NOTIFY progressChanged)
    Q_PROPERTY(int lengthSeconds READ lengthSeconds NOTIFY progressChanged)
    Q_PROPERTY(QString progressText READ progressText NOTIFY progressChanged)
    Q_PROPERTY(QString lengthText READ lengthText NOTIFY progressChanged)
    Q_PROPERTY(QVariantList devices READ devices NOTIFY devicesChanged)
    Q_PROPERTY(QString logs READ logs NOTIFY logsChanged)
    Q_PROPERTY(bool running READ running NOTIFY runningChanged)

public:
    explicit AppController(QObject *parent = nullptr);
    ~AppController() override;

    QString receiverName() const { return m_receiverName; }
    QString provider() const { return m_provider; }
    QString title() const { return m_title; }
    QString artist() const { return m_artist; }
    QString album() const { return m_album; }
    QString genre() const { return m_genre; }
    QString albumArt() const { return m_albumArt; }
    QString remoteAlbumArt() const { return m_remoteAlbumArt; }
    int progressSeconds() const { return m_progressSeconds; }
    int lengthSeconds() const { return m_lengthSeconds; }
    QString progressText() const;
    QString lengthText() const;
    QVariantList devices() const { return m_devices; }
    QString logs() const { return m_logs; }
    bool running() const;
    bool playing() const { return m_playing; }

    void setReceiverName(const QString &value);
    void setProvider(const QString &value);

    Q_INVOKABLE void saveSettings();
    Q_INVOKABLE void startReceiver();
    Q_INVOKABLE void restartReceiver();
    Q_INVOKABLE void stopReceiver();
    Q_INVOKABLE void playbackControl(const QString &control);
    Q_INVOKABLE void forgetDevice(const QString &deviceId);

signals:
    void settingsChanged();
    void nowPlayingChanged();
    void progressChanged();
    void devicesChanged();
    void logsChanged();
    void runningChanged();

private slots:
    void readStandardOutput();
    void readStandardError();
    void receiverFinished(int exitCode, QProcess::ExitStatus status);
    void handleMetadata(const QString &kind, const QString &code, const QByteArray &data);

private:
    void appendLog(const QString &line);
    void parseReceiverLine(const QString &line);
    void updateDevice(const QString &id, const QString &name);
    void loadAlbumArt();
    void uploadAlbumArt(const QByteArray &data);
    QString appSecret() const;
    bool sendDacpControl(const QString &command);
    void sendDbusControl(const QString &method);
    void sanitizeAppImageEnvironment();
    void resetSession();
    bool executableAvailable(const QString &name) const;
    bool prepareMetadataPipe();
    QString configPath(const QString &file = {}) const;
    static int parseTime(const QString &value);
    static QString formatTime(int seconds);

    QProcess m_receiver;
    MetadataReader *m_metadataReader = nullptr;
    QString m_receiverName = QStringLiteral("UiPlay");
    QString m_provider = QStringLiteral("shairport");
    QString m_title;
    QString m_artist;
    QString m_album;
    QString m_genre;
    QString m_albumArt;
    int m_progressSeconds = 0;
    int m_lengthSeconds = 0;
    QVariantList m_devices;
    QString m_logs;
    QString m_socket;
    QString m_currentDeviceId;
    QString m_clientIp;
    quint16 m_dacpPort = 0;
    QString m_activeRemote;
    QByteArray m_stdoutBuffer;
    quint64 m_artRevision = 0;
    QString m_remoteAlbumArt;
    bool m_playing = true;
    QByteArray m_lastArtHash;
    QString m_authToken;
    QNetworkAccessManager m_network;
    QNetworkAccessManager m_dacpNetwork;
    MprisService *m_mpris = nullptr;
    DiscordRpc *m_discord = nullptr;
};
