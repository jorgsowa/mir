===config===
find_dead_code=true
===file===
<?php
class Foo {
    public string $name = 'bar';
}
===expect===
