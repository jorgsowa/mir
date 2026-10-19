===description===
A method's by-ref param defines the out variable in the true branch of `&&`,
including through a property receiver and an assignment wrapped in `!`.
===file===
<?php
interface Marshaller {
    /** @param array<string, mixed> $values */
    public function marshall(array $values, ?array &$failed): array;
}

final class Adapter {
    public function __construct(private ?Marshaller $marshaller) {}

    /** @param array<string, mixed> $values */
    public function viaProperty(array $values): ?array {
        if (null !== $this->marshaller && (!$values = $this->marshaller->marshall($values, $failed))) {
            /** @mir-check $failed is array|null */
            return $failed;
        }
        return null;
    }

    /** @param array<string, mixed> $values */
    public static function viaVariable(array $values, ?Marshaller $m): ?array {
        if ($m && ($marshalled = $m->marshall($values, $failed))) {
            return $failed ?? $marshalled;
        }
        return null;
    }
}
