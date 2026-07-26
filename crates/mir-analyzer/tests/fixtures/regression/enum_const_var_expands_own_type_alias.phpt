===description===
An enum constant's own `@var` annotation never expanded the enum's own
`@psalm-type` alias, the same gap as class/trait/interface members — the
constant resolved to the literal, nonexistent class `Payload` instead of
the aliased shape.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type Payload = array{id: int, name: string}
 */
enum Status {
    case Active;

    /** @var Payload */
    const DEFAULT_PAYLOAD = ['id' => 0, 'name' => ''];

    public function check(): void {
        /** @mir-check self::DEFAULT_PAYLOAD is array{id: int, name: string} */
        echo "ok";
    }
}
===expect===
