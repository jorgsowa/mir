===description===
does not report public method
===config===
find_dead_code=true
===file===
<?php
class Foo {
    public function publicMethod(): void {}
}
===expect===
===ignore===
TODO
