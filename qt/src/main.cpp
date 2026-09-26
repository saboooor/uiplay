#include "appcontroller.h"

#include <QApplication>
#include <QLibraryInfo>
#include <QQuickStyle>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QDir>
#include <QStandardPaths>

int main(int argc, char *argv[])
{
    const QString desktop = qEnvironmentVariable("XDG_CURRENT_DESKTOP");
    const bool isPlasma = desktop.contains(QStringLiteral("KDE"), Qt::CaseInsensitive)
                          || qEnvironmentVariableIsSet("KDE_FULL_SESSION");
    const QString kdeStyle = QLibraryInfo::path(QLibraryInfo::QmlImportsPath)
                             + QStringLiteral("/org/kde/desktop");
    if (isPlasma && QDir(kdeStyle).exists()) {
        // Platform themes are selected while QGuiApplication is constructed.
        // Plasma normally exports this itself, but launchers, IDEs, and shells
        // do not always inherit it.
        if (!qEnvironmentVariableIsSet("QT_QPA_PLATFORMTHEME"))
            qputenv("QT_QPA_PLATFORMTHEME", QByteArrayLiteral("kde"));
        QQuickStyle::setStyle(QStringLiteral("org.kde.desktop"));
    }

    // KDE's desktop Controls style paints through the active Qt Widgets style.
    // QApplication is therefore required even though UiPlay's UI is Qt Quick.
    QApplication app(argc, argv);
    QApplication::setApplicationName(QStringLiteral("UiPlay"));
    QApplication::setOrganizationName(QStringLiteral("Luminescent"));
    QApplication::setApplicationVersion(QStringLiteral(UIPLAY_VERSION));
    if (!QStandardPaths::locate(QStandardPaths::ApplicationsLocation,
            QStringLiteral("ca.saboor.uiplay.desktop")).isEmpty())
        QApplication::setDesktopFileName(QStringLiteral("ca.saboor.uiplay"));

    AppController controller;
    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("appController"), &controller);
    engine.loadFromModule(QStringLiteral("UiPlay"), QStringLiteral("Main"));
    if (engine.rootObjects().isEmpty())
        return -1;

    controller.startReceiver();
    return app.exec();
}
