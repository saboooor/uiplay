#include "discordrpc.h"
#include "appcontroller.h"

#include <QCoreApplication>
#include <QDateTime>
#include <QDir>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStandardPaths>
#include <QUuid>
#include <QtEndian>

static constexpr auto discordApplicationId = "1397877327622311997";

DiscordRpc::DiscordRpc(AppController *controller, QObject *parent)
    : QObject(parent), m_controller(controller)
{
    m_reconnect.setInterval(10000);
    connect(&m_reconnect, &QTimer::timeout, this, &DiscordRpc::connectDiscord);
    connect(&m_socket, &QLocalSocket::connected, this, [this] {
        sendFrame(0, QJsonObject{{QStringLiteral("v"), 1},
                                 {QStringLiteral("client_id"), QString::fromLatin1(discordApplicationId)}});
        updateActivity();
        m_reconnect.stop();
    });
    connect(&m_socket, &QLocalSocket::disconnected, &m_reconnect, qOverload<>(&QTimer::start));
    connect(&m_socket, &QLocalSocket::errorOccurred, this, [this] {
        if (m_socket.state() == QLocalSocket::UnconnectedState && ++m_endpoint < 10)
            connectDiscord();
        else
            m_reconnect.start();
    });
    connect(controller, &AppController::nowPlayingChanged, this, &DiscordRpc::updateActivity);
    connect(controller, &AppController::progressChanged, this, &DiscordRpc::updateActivity);
    QTimer::singleShot(0, this, &DiscordRpc::connectDiscord);
}

void DiscordRpc::connectDiscord()
{
    if (m_socket.state() != QLocalSocket::UnconnectedState)
        return;
    if (m_endpoint >= 10)
        m_endpoint = 0;
    QString runtime = qEnvironmentVariable("XDG_RUNTIME_DIR");
    if (runtime.isEmpty())
        runtime = QDir::tempPath();
    const QString path = runtime + QStringLiteral("/discord-ipc-%1").arg(m_endpoint);
    m_socket.connectToServer(path);
}

void DiscordRpc::sendFrame(quint32 opcode, const QJsonObject &object)
{
    if (m_socket.state() != QLocalSocket::ConnectedState)
        return;
    const QByteArray payload = QJsonDocument(object).toJson(QJsonDocument::Compact);
    QByteArray frame;
    frame.resize(8);
    qToLittleEndian(opcode, frame.data());
    qToLittleEndian(quint32(payload.size()), frame.data() + 4);
    frame += payload;
    m_socket.write(frame);
}

void DiscordRpc::updateActivity()
{
    if (m_socket.state() != QLocalSocket::ConnectedState)
        return;
    QJsonObject activity{{QStringLiteral("type"), 2},
                         {QStringLiteral("details"), m_controller->title()},
                         {QStringLiteral("state"), m_controller->artist()}};
    QJsonObject assets{{QStringLiteral("small_image"), QStringLiteral("icon")},
                       {QStringLiteral("small_text"), QStringLiteral("UiPlay")},
                       {QStringLiteral("large_text"), m_controller->album()}};
    const QString art = m_controller->remoteAlbumArt();
    if (!art.isEmpty())
        assets[QStringLiteral("large_image")] = art;
    activity[QStringLiteral("assets")] = assets;
    if (m_controller->lengthSeconds() > 0) {
        const qint64 now = QDateTime::currentSecsSinceEpoch();
        const qint64 start = now - m_controller->progressSeconds();
        activity[QStringLiteral("timestamps")] = QJsonObject{{QStringLiteral("start"), start},
                                                               {QStringLiteral("end"), start + m_controller->lengthSeconds()}};
    }
    sendFrame(1, QJsonObject{{QStringLiteral("cmd"), QStringLiteral("SET_ACTIVITY")},
                              {QStringLiteral("args"), QJsonObject{{QStringLiteral("pid"), QCoreApplication::applicationPid()},
                                                                    {QStringLiteral("activity"), activity}}},
                              {QStringLiteral("nonce"), QUuid::createUuid().toString(QUuid::WithoutBraces)}});
}
