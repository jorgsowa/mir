===description===
@param-out on a static method: the out-type is written back to the caller's
variable after the static call.
===config===
suppress=UnusedVariable
===file===
<?php
class Registry {
    /**
     * @param-out array<string, int> $map
     */
    public static function build(array $items, mixed &$map): void {
        $map = array_flip($items);
    }
}

Registry::build(["a", "b", "c"], $result);
/** @mir-check $result is array<string, int> */
$_ = $result;
===expect===
