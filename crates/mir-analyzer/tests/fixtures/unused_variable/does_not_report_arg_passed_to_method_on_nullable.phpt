===source===
<?php
class Baz {
    public function handle(?object $obj): void {
        $ctx = ['key' => 'value'];
        $obj->doSomething($ctx);
    }
}
===expect===
PossiblyNullMethodCall: $obj->doSomething($ctx)
