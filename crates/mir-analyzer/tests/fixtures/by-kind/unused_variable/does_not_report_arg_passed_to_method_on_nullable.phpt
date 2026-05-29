===description===
does not report arg passed to method on nullable
===file===
<?php
class Baz {
    public function handle(?object $obj): void {
        $ctx = ['key' => 'value'];
        $obj->doSomething($ctx);
    }
}
===expect===
PossiblyNullMethodCall@5:9-5:32: Cannot call method doSomething() on possibly null value
