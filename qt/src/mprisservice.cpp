#include "mprisservice.h"
#include "appcontroller.h"

#include <QApplication>
#include <QDBusAbstractAdaptor>
#include <QDBusConnection>
#include <QDBusMessage>
#include <QDBusObjectPath>
#include <QVariantMap>

static constexpr auto serviceName = "org.mpris.MediaPlayer2.UiPlay";
static constexpr auto objectPath = "/org/mpris/MediaPlayer2";

class MprisRootAdaptor final : public QDBusAbstractAdaptor
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.mpris.MediaPlayer2")
    Q_PROPERTY(bool CanQuit READ canQuit CONSTANT)
    Q_PROPERTY(bool CanRaise READ canRaise CONSTANT)
    Q_PROPERTY(bool HasTrackList READ hasTrackList CONSTANT)
    Q_PROPERTY(QString Identity READ identity CONSTANT)
    Q_PROPERTY(QString DesktopEntry READ desktopEntry CONSTANT)
    Q_PROPERTY(QStringList SupportedUriSchemes READ supportedUriSchemes CONSTANT)
    Q_PROPERTY(QStringList SupportedMimeTypes READ supportedMimeTypes CONSTANT)
public:
    explicit MprisRootAdaptor(QObject *parent) : QDBusAbstractAdaptor(parent) {}
    bool canQuit() const { return true; }
    bool canRaise() const { return false; }
    bool hasTrackList() const { return false; }
    QString identity() const { return QStringLiteral("UiPlay"); }
    QString desktopEntry() const { return QStringLiteral("uiplay"); }
    QStringList supportedUriSchemes() const { return {}; }
    QStringList supportedMimeTypes() const { return {}; }
public slots:
    void Raise() {}
    void Quit() { QApplication::quit(); }
};

class MprisPlayerAdaptor final : public QDBusAbstractAdaptor
{
    Q_OBJECT
    Q_CLASSINFO("D-Bus Interface", "org.mpris.MediaPlayer2.Player")
    Q_PROPERTY(QString PlaybackStatus READ playbackStatus)
    Q_PROPERTY(QString LoopStatus READ loopStatus)
    Q_PROPERTY(double Rate READ rate)
    Q_PROPERTY(bool Shuffle READ shuffle)
    Q_PROPERTY(QVariantMap Metadata READ metadata)
    Q_PROPERTY(double Volume READ volume)
    Q_PROPERTY(qlonglong Position READ position)
    Q_PROPERTY(double MinimumRate READ minimumRate)
    Q_PROPERTY(double MaximumRate READ maximumRate)
    Q_PROPERTY(bool CanGoNext READ canControlTrack)
    Q_PROPERTY(bool CanGoPrevious READ canControlTrack)
    Q_PROPERTY(bool CanPlay READ canControlTrack)
    Q_PROPERTY(bool CanPause READ canControlTrack)
    Q_PROPERTY(bool CanSeek READ canSeek)
    Q_PROPERTY(bool CanControl READ canControl CONSTANT)
public:
    MprisPlayerAdaptor(AppController *controller, QObject *parent)
        : QDBusAbstractAdaptor(parent), m_controller(controller) {}
    QString playbackStatus() const {
        return m_controller->title().isEmpty() ? QStringLiteral("Stopped")
             : m_controller->playing() ? QStringLiteral("Playing") : QStringLiteral("Paused");
    }
    QString loopStatus() const { return QStringLiteral("None"); }
    double rate() const { return 1.0; }
    bool shuffle() const { return false; }
    QVariantMap metadata() const {
        QVariantMap values;
        values[QStringLiteral("mpris:trackid")] = QVariant::fromValue(QDBusObjectPath(QStringLiteral("/org/mpris/MediaPlayer2/Track/Current")));
        values[QStringLiteral("xesam:title")] = m_controller->title();
        values[QStringLiteral("xesam:album")] = m_controller->album();
        values[QStringLiteral("xesam:artist")] = QStringList{m_controller->artist()};
        values[QStringLiteral("xesam:genre")] = QStringList{m_controller->genre()};
        if (m_controller->lengthSeconds() > 0)
            values[QStringLiteral("mpris:length")] = qlonglong(m_controller->lengthSeconds()) * 1000000;
        const QString art = m_controller->remoteAlbumArt().isEmpty() ? m_controller->albumArt() : m_controller->remoteAlbumArt();
        if (!art.isEmpty())
            values[QStringLiteral("mpris:artUrl")] = art;
        return values;
    }
    double volume() const { return 1.0; }
    qlonglong position() const { return qlonglong(m_controller->progressSeconds()) * 1000000; }
    double minimumRate() const { return 1.0; }
    double maximumRate() const { return 1.0; }
    bool canControlTrack() const { return !m_controller->title().isEmpty(); }
    bool canSeek() const { return false; }
    bool canControl() const { return true; }
public slots:
    void Next() { m_controller->playbackControl(QStringLiteral("next")); }
    void Previous() { m_controller->playbackControl(QStringLiteral("previous")); }
    void Pause() { if (m_controller->playing()) m_controller->playbackControl(QStringLiteral("playpause")); }
    void PlayPause() { m_controller->playbackControl(QStringLiteral("playpause")); }
    void Stop() { if (m_controller->playing()) m_controller->playbackControl(QStringLiteral("playpause")); }
    void Play() { if (!m_controller->playing()) m_controller->playbackControl(QStringLiteral("playpause")); }
    void Seek(qlonglong) {}
    void SetPosition(const QDBusObjectPath &, qlonglong) {}
    void OpenUri(const QString &) {}
signals:
    void Seeked(qlonglong position);
private:
    AppController *m_controller;
};

MprisService::MprisService(AppController *controller, QObject *parent)
    : QObject(parent), m_controller(controller)
{
    new MprisRootAdaptor(this);
    new MprisPlayerAdaptor(controller, this);
    auto bus = QDBusConnection::sessionBus();
    bus.registerObject(QString::fromLatin1(objectPath), this, QDBusConnection::ExportAdaptors);
    bus.registerService(QString::fromLatin1(serviceName));
    connect(controller, &AppController::nowPlayingChanged, this, &MprisService::notifyProperties);
    connect(controller, &AppController::progressChanged, this, &MprisService::notifyProperties);
}

void MprisService::notifyProperties()
{
    QVariantMap changed;
    const bool hasTrack = !m_controller->title().isEmpty();
    changed[QStringLiteral("PlaybackStatus")] = !hasTrack ? QStringLiteral("Stopped")
        : m_controller->playing() ? QStringLiteral("Playing") : QStringLiteral("Paused");
    QVariantMap metadata;
    metadata[QStringLiteral("mpris:trackid")] = QVariant::fromValue(
        QDBusObjectPath(QStringLiteral("/org/mpris/MediaPlayer2/Track/Current")));
    metadata[QStringLiteral("xesam:title")] = m_controller->title();
    metadata[QStringLiteral("xesam:album")] = m_controller->album();
    metadata[QStringLiteral("xesam:artist")] = QStringList{m_controller->artist()};
    metadata[QStringLiteral("xesam:genre")] = QStringList{m_controller->genre()};
    if (m_controller->lengthSeconds() > 0)
        metadata[QStringLiteral("mpris:length")] = qlonglong(m_controller->lengthSeconds()) * 1000000;
    const QString art = m_controller->remoteAlbumArt().isEmpty()
                            ? m_controller->albumArt() : m_controller->remoteAlbumArt();
    if (!art.isEmpty())
        metadata[QStringLiteral("mpris:artUrl")] = art;
    changed[QStringLiteral("Metadata")] = metadata;
    changed[QStringLiteral("Position")] = qlonglong(m_controller->progressSeconds()) * 1000000;
    changed[QStringLiteral("CanGoNext")] = hasTrack;
    changed[QStringLiteral("CanGoPrevious")] = hasTrack;
    changed[QStringLiteral("CanPlay")] = hasTrack;
    changed[QStringLiteral("CanPause")] = hasTrack;
    QDBusMessage signal = QDBusMessage::createSignal(QString::fromLatin1(objectPath),
        QStringLiteral("org.freedesktop.DBus.Properties"), QStringLiteral("PropertiesChanged"));
    signal << QStringLiteral("org.mpris.MediaPlayer2.Player") << changed << QStringList{};
    QDBusConnection::sessionBus().send(signal);
}

#include "mprisservice.moc"
