===description===
does not report arg passed to method on null
===file===
<?php
class Bar {
    public function handle(?object $obj): void {
        $ctx = ['key' => 'value'];
        if ($obj === null) {
            $obj->doSomething($ctx);
        }
    }
}
===expect===
NullMethodCall@6:12: Cannot call method doSomething() on null
