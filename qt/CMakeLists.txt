cmake_minimum_required(VERSION 3.21)

project(UiPlayQt VERSION 1.0.0 LANGUAGES CXX)

set(UIPLAY_VERSION "1.0.0-alpha.1")

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Gui Widgets Quick Qml QuickControls2 Network DBus)
qt_standard_project_setup(REQUIRES 6.5)

qt_add_executable(uiplay-qt
    src/main.cpp
    src/appcontroller.cpp
    src/appcontroller.h
    src/metadatareader.cpp
    src/metadatareader.h
    src/mprisservice.cpp
    src/mprisservice.h
    src/discordrpc.cpp
    src/discordrpc.h
)

qt_add_qml_module(uiplay-qt
    URI UiPlay
    VERSION 1.0
    QML_FILES qml/Main.qml
)

target_link_libraries(uiplay-qt PRIVATE
    Qt6::Core Qt6::Gui Qt6::Widgets Qt6::Quick Qt6::Qml Qt6::QuickControls2 Qt6::Network Qt6::DBus
)
target_compile_definitions(uiplay-qt PRIVATE UIPLAY_VERSION="${UIPLAY_VERSION}")

install(TARGETS uiplay-qt
    BUNDLE DESTINATION .
    RUNTIME DESTINATION bin
)
install(FILES packaging/ca.saboor.uiplay.desktop
    DESTINATION share/applications
)
install(FILES packaging/ca.saboor.uiplay.metainfo.xml
    DESTINATION share/metainfo
)
install(FILES ../src-tauri/icons/icon.png
    DESTINATION share/icons/hicolor/512x512/apps
    RENAME ca.saboor.uiplay.png
)

set(CPACK_PACKAGE_NAME "uiplay-qt")
set(CPACK_PACKAGE_VENDOR "LuminescentDev")
set(CPACK_PACKAGE_CONTACT "LuminescentDev")
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY "Native AirPlay receiver with MPRIS and Discord Rich Presence")
set(CPACK_PACKAGE_VERSION "${UIPLAY_VERSION}")
set(CPACK_PACKAGE_FILE_NAME "uiplay-qt-${UIPLAY_VERSION}-${CMAKE_SYSTEM_PROCESSOR}")
set(CPACK_GENERATOR "DEB")
set(CPACK_DEBIAN_PACKAGE_NAME "uiplay-qt")
set(CPACK_DEBIAN_PACKAGE_VERSION "1.0.0~alpha1")
set(CPACK_DEBIAN_FILE_NAME DEB-DEFAULT)
set(CPACK_DEBIAN_PACKAGE_SECTION "sound")
set(CPACK_DEBIAN_PACKAGE_SHLIBDEPS ON)
set(CPACK_DEBIAN_PACKAGE_RECOMMENDS "shairport-sync, avahi-utils")
include(CPack)
