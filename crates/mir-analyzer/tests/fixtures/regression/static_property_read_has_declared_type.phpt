===description===
Reading a static property (`self::$prop`, `Class::$prop`) must use its
declared type instead of unconditionally returning `mixed` —
analyze_static_property_access previously ignored the property's type
entirely. Also covers the companion narrowing gap this fix would otherwise
expose: an assignment inside a null-check guard (the classic lazy-init
pattern) must be reflected in later reads of the same static property.
===config===
suppress=MissingPropertyType
===file===
<?php

class Counter {
    public static int $count = 0;

    public static function get(): int {
        $x = self::$count;
        /** @mir-check $x is int */
        return $x;
    }
}

class MimeTypes {}
class MimeType {
    /** @var MimeTypes|null */
    private static $mime;

    public static function getMimeTypes(): MimeTypes {
        if (self::$mime === null) {
            self::$mime = new MimeTypes();
        }
        return self::$mime;
    }
}

function useCounter(): int {
    return Counter::$count;
}
===expect===
