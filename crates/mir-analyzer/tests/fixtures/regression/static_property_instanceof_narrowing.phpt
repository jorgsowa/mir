===description===
`self::$prop instanceof ClassName` / `static::$prop instanceof ClassName` must
narrow the static property's type in the guarded branch, the same way
`$this->prop instanceof ClassName` already does for instance properties —
narrowing.rs's Instanceof arm previously only checked extract_var_name and
extract_prop_access, never extract_static_prop_access, so the guard had no
effect and the property stayed at its declared (nullable) type.
===config===
suppress=MissingPropertyType
===file===
<?php

interface Logger {
    public function flush(): void;
}
class FileLogger implements Logger {
    public function flush(): void {}
    public function rotate(): void {}
}

class Service {
    protected static ?Logger $logger = null;

    public static function useIt(): void {
        if (self::$logger instanceof FileLogger) {
            self::$logger->rotate();
        }
    }

    public static function useItNegated(): void {
        if (!(self::$logger instanceof FileLogger)) {
            return;
        }
        self::$logger->rotate();
    }
}

class Child extends Service {
    public static function useItViaStatic(): void {
        if (static::$logger instanceof FileLogger) {
            static::$logger->rotate();
        }
    }
}
===expect===
