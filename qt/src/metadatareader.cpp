#include "metadatareader.h"

#include <QFile>
#include <QRegularExpression>

MetadataReader::MetadataReader(QString pipePath, QObject *parent)
    : QThread(parent), m_pipePath(std::move(pipePath))
{
}

MetadataReader::~MetadataReader()
{
    stop();
}

void MetadataReader::stop()
{
    requestInterruption();
    QFile wake(m_pipePath);
    if (wake.open(QIODevice::WriteOnly | QIODevice::Unbuffered))
        wake.write("\n");
    wait(2000);
}

static QString xmlValue(const QString &text, const QString &tag)
{
    const QRegularExpression expression(QStringLiteral("<%1>([^<]*)</%1>").arg(tag));
    return expression.match(text).captured(1);
}

static QString decodeCode(const QString &hex)
{
    const QByteArray bytes = QByteArray::fromHex(hex.toLatin1());
    return QString::fromLatin1(bytes);
}

void MetadataReader::run()
{
    QFile pipe(m_pipePath);
    if (!pipe.open(QIODevice::ReadOnly | QIODevice::Unbuffered)) {
        emit error(QStringLiteral("Failed to open metadata pipe: %1").arg(pipe.errorString()));
        return;
    }

    while (!isInterruptionRequested()) {
        const QByteArray headerBytes = pipe.readLine();
        if (headerBytes.isEmpty()) {
            msleep(20);
            continue;
        }
        const QString header = QString::fromUtf8(headerBytes);
        const QString kind = decodeCode(xmlValue(header, QStringLiteral("type")));
        const QString code = decodeCode(xmlValue(header, QStringLiteral("code")));
        const qsizetype expected = xmlValue(header, QStringLiteral("length")).toLongLong();
        if (kind.isEmpty() || code.isEmpty())
            continue;
        if (expected == 0) {
            emit item(kind, code, {});
            continue;
        }

        QByteArray encoded;
        QByteArray line = pipe.readLine();
        if (!line.contains("<data encoding=\"base64\">"))
            continue;
        while (!isInterruptionRequested()) {
            line = pipe.readLine();
            const qsizetype end = line.indexOf("</data></item>");
            encoded += end >= 0 ? line.first(end) : line;
            if (end >= 0)
                break;
        }
        encoded.replace("\n", "");
        encoded.replace("\r", "");
        const QByteArray data = QByteArray::fromBase64(encoded);
        if (data.size() == expected)
            emit item(kind, code, data);
        else
            emit error(QStringLiteral("Invalid metadata item length"));
    }
}
