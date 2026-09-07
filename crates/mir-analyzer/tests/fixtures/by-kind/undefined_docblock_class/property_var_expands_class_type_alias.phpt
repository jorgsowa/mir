===description===
A class (or trait) property's own `@var` annotation never expanded the
declaring class-like's own `@psalm-type` alias — unlike `@param`/`@return`
and an inline local `@var`, which already did. The property resolved to
the literal, nonexistent class `Payload` instead of the aliased shape.
===config===
suppress=UnusedParam,UnusedVariable,MissingConstructor,MissingPropertyType
===file===
<?php
/**
 * @psalm-type Payload = array{id: int, name: string}
 */
class Holder {
    /** @var Payload */
    private $data;

    public function id(): int {
        /** @mir-check $this->data is array{id: int, name: string} */
        echo "ok";
        return $this->data['id'];
    }
}

/**
 * @psalm-type Payload = array{id: int, name: string}
 */
trait HolderTrait {
    /** @var Payload */
    private $traitData;
}

class UsesTrait {
    use HolderTrait;

    public function check(): void {
        /** @mir-check $this->traitData is array{id: int, name: string} */
        echo "ok";
    }
}
===expect===
