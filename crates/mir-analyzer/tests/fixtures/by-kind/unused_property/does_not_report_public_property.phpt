===description===
does not report public property
===config===
suppress=
===file===
<?php
class Foo {
    public string $name = 'bar';
}
===expect===
