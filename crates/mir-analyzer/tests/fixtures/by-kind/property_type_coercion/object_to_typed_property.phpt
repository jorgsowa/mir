===description===
An `object` assigned to a class-typed property is a coercion, and the property then holds the declared class.
===file===
<?php
interface Info {}

final class Registry {
    public static function service(string $id): object { return new $id(); }
}

class Holder {
    private Info $info;
    protected ?Info $lazy = null;
    private int $count = 0;

    public function __construct(object $o) {
        $this->info = $o;
//      ^^^^^^^^^^^^^^^^ PropertyTypeCoercion: Property $info expects 'Info', cannot assign 'object' — coercion may fail at runtime
    }

    public function lazy(): Info {
        if (!$this->lazy) {
            $this->lazy = Registry::service('info');
//          ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ PropertyTypeCoercion: Property $lazy expects 'Info|null', cannot assign 'object' — coercion may fail at runtime
        }
        /** @mir-check $this->lazy is Info */
        echo 1;
        return $this->lazy;
    }

    public function scalar(object $o): void {
        $this->count = $o;
//      ^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $count expects 'int', cannot assign 'object'
    }
}
