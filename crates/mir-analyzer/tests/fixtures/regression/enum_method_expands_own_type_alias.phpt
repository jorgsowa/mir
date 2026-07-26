===description===
An enum-declared `@psalm-type` alias was never wired into the enum's own
methods — `build_method_storage` was always called with `aliases: None`
for an enum, unlike class/interface/trait, and the enum's own alias table
was computed AFTER the member loop had already processed every method.
A method's `@param` referencing the alias resolved to the literal,
nonexistent class `Payload` instead of the aliased shape.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @psalm-type Payload = array{id: int, name: string}
 */
enum Status {
    case Active;
    case Inactive;

    /** @param Payload $p */
    public function describe($p): void {
        /** @mir-check $p is array{id: int, name: string} */
        echo "ok";
    }
}
===expect===
