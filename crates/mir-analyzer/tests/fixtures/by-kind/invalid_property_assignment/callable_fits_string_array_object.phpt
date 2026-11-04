===description===
A callable is a string, an array or an object, so it fits a property typed as their union
===file===
<?php
final class Holder {
    private string|array|object $any = '';
    private int|string $scalar = 0;

    public function set(callable $c): void {
        $this->any = $c;
        $this->scalar = $c;
//      ^^^^^^^^^^^^^^^^^^ InvalidPropertyAssignment: Property $scalar expects 'int|string', cannot assign 'callable'
    }
}
