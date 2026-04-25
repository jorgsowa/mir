===file===
<?php
class Qux {
    public function handle(?object $obj): void {
        $ctx = ['key' => 'value'];
        $obj?->doSomething($ctx);
    }
}
===expect===
