#pragma once

#include <QLocalSocket>
#include <QObject>
#include <QTimer>

class AppController;

class DiscordRpc final : public QObject
{
    Q_OBJECT
public:
    explicit DiscordRpc(AppController *controller, QObject *parent = nullptr);

private:
    void connectDiscord();
    void sendFrame(quint32 opcode, const QJsonObject &object);
    void updateActivity();
    AppController *m_controller;
    QLocalSocket m_socket;
    QTimer m_reconnect;
    int m_endpoint = 0;
};
