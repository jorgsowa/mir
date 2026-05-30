===description===
does not report public method
===config===
suppress=
===file===
<?php
class Foo {
    public function publicMethod(): void {}
}
===expect===
