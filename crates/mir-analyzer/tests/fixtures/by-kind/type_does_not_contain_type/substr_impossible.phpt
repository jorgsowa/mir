===description===
Substr impossible
===file===
<?php
class HelloWorld
{
    public function sayHello(string $s): void
    {
        if (substr($s, 0, 6) === "abc") {}
    }
}
===expect===
