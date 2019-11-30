#include <QtGui/QIcon>
#include <QtQuick/QQuickImageProvider>

class IconProvider : public QQuickImageProvider {
    public:
        IconProvider() : QQuickImageProvider(QQuickImageProvider::Pixmap) {}

        QPixmap requestPixmap(const QString &id, QSize *size, const QSize &requestedSize) override {
            int width = 128;
            int height = 128;

            if (size)
                *size = QSize(width, height);


            QIcon icon = QIcon::fromTheme(id);
            return icon.pixmap(requestedSize.width() > 0 ? requestedSize.width() : width,
                      requestedSize.height() > 0 ? requestedSize.height() : height);
        }

};