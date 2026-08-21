query = 'SELECT reltuples FROM pg_class WHERE relname = ' +
  "'#{table_name}'"

execute("ALTER TABLE #{table} RENAME CONSTRAINT " +
  "#{old_name} TO #{new_name}")

script = "selection.addRange(range);" + "const event = new PointerEvent('pointerup');" +
  "document.dispatchEvent(event);"
