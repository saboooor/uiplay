#pragma once

#include <QObject>

class AppController;

class MprisService final : public QObject
{
    Q_OBJECT
public:
    explicit MprisService(AppController *controller, QObject *parent = nullptr);
    void notifyProperties();

private:
    AppController *m_controller;
};
