#pragma once

#include <QThread>

class MetadataReader final : public QThread
{
    Q_OBJECT
public:
    explicit MetadataReader(QString pipePath, QObject *parent = nullptr);
    ~MetadataReader() override;
    void stop();

signals:
    void item(QString kind, QString code, QByteArray data);
    void error(QString message);

protected:
    void run() override;

private:
    QString m_pipePath;
};
