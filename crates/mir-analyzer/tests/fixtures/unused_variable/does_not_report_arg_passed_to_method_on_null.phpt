===source===
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
NullMethodCall: Cannot call method doSomething() on null
