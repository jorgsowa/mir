===description===
An interface constant's own `@var` annotation never expanded the
declaring interface's own `@psalm-type` alias, the same gap as class/trait
properties. The constant resolved to the literal, nonexistent class
`Payload` instead of the aliased shape.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/**
 * @psalm-type Payload = array{id: int, name: string}
 */
interface HasDefault {
    /** @var Payload */
    const DEFAULT_PAYLOAD = ['id' => 0, 'name' => ''];
}

class Impl implements HasDefault {
    public function check(): void {
        /** @mir-check self::DEFAULT_PAYLOAD is array{id: int, name: string} */
        echo "ok";
    }
}
===expect===
