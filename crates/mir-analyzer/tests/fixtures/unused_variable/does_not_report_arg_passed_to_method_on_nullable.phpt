===file===
<?php
class Baz {
    public function handle(?object $obj): void {
        $ctx = ['key' => 'value'];
        $obj->doSomething($ctx);
    }
}
===expect===
PossiblyNullMethodCall: Cannot call method doSomething() on possibly null value
