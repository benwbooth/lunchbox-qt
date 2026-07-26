#include "host_screens.h"

#include <QtGui/QGuiApplication>
#include <QtGui/QScreen>
#include <QtGui/QWindow>

namespace {
QScreen* screen_at(std::int32_t index)
{
  const auto screens = QGuiApplication::screens();
  return index >= 0 && index < screens.size() ? screens.at(index) : nullptr;
}
} // namespace

std::int32_t host_screen_count()
{
  return QGuiApplication::screens().size();
}

QString host_screen_name_at(std::int32_t index)
{
  const auto* screen = screen_at(index);
  return screen == nullptr ? QString{} : screen->name();
}

std::int32_t host_screen_width_at(std::int32_t index)
{
  const auto* screen = screen_at(index);
  return screen == nullptr ? 0 : screen->geometry().width();
}

std::int32_t host_screen_height_at(std::int32_t index)
{
  const auto* screen = screen_at(index);
  return screen == nullptr ? 0 : screen->geometry().height();
}

bool route_window_to_host_screen(QObject* object, std::int32_t index)
{
  auto* window = qobject_cast<QWindow*>(object);
  auto* screen = screen_at(index);
  if (window == nullptr || screen == nullptr) {
    return false;
  }
  window->setScreen(screen);
  return window->screen() == screen;
}
