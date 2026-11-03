===description===
An inline `@var` over `$x = $x->f()` types the assigned result; the right-hand side still sees the old `$x`.
===file===
<?php
#[\Attribute]
final class Subscribed {
    public ?string $key = null;
}

final class Locator {
    public static function keys(\ReflectionMethod $method): ?string {
        foreach ($method->getAttributes() as $attribute) {
            /** @var Subscribed $attribute */
            $attribute = $attribute->newInstance();
            return $attribute->key;
        }
        return null;
    }
}
