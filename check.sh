#!/usr/bin/env bash

# Включаем режим завершения при ошибках
set -eo pipefail

# Целевой таргет без поддержки std
TARGET="thumbv7m-none-eabi"

echo "🔍 Поиск локальных no_std пакетов в директории core/..."

# 1. Анализируем массив пакетов (.packages).
# 2. Оставляем только те, что лежат физически в папке /core/.
# 3. ИСКЛЮЧАЕМ пакеты, у которых тип таргета (kind) содержит "proc-macro".
# 4. Забираем чистые имена пакетов.
packages=$(cargo metadata --format-version 1 --no-deps | jq -r '
  .packages[]

  | select(.manifest_path | contains("/core/"))
  | select([.targets[].kind[]] | contains(["proc-macro"]) | not)
  | .name
')

# Проверяем, удалось ли найти валидные пакеты в core/
if [ -z "$packages" ]; then
    echo "⚠️ В core/ не найдено no_std пакетов (или там остались только proc-macro)."
    exit 0
fi

# Убеждаемся, что таргет установлен в системе перед проверкой
if ! rustup target list | grep -q "$TARGET (installed)"; then
    echo "📥 Установка таргета $TARGET..."
    rustup target add "$TARGET"
fi

# Итерация по каждому no_std пакету из core/
for pkg in $packages; do
    echo "--------------------------------------------------"
    echo "🔨 Запуск cargo check для [$pkg] ($TARGET)..."
    echo "--------------------------------------------------"

    # Запускаем проверку пакета под no_std таргет.
    cargo check -p "$pkg" --target "$TARGET" || {
        echo -e "\n❌ Ошибка! Пакет [$pkg] содержит std-зависимости или не компилируется."
        echo "🛑 Итерация прервана."
        exit 1
    }
done

echo "--------------------------------------------------"
echo "✅ Все no_std библиотеки из core/ успешно проверены под $TARGET!"
