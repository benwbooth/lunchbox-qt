#pragma once

#include <QtCore/QObject>
#include <QtCore/QString>

#include <cstdint>

std::int32_t host_screen_count();
QString host_screen_name_at(std::int32_t index);
std::int32_t host_screen_width_at(std::int32_t index);
std::int32_t host_screen_height_at(std::int32_t index);
bool route_window_to_host_screen(QObject* object, std::int32_t index);
