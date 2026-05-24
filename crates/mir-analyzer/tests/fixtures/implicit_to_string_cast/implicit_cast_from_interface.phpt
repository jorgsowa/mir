===description===
implicitCastFromInterface
===file===
<?php
interface I {
    public function __toString();
}

/** @psalm-mutation-free */
function takesString(string $str): void { }

function takesI(I $i): void
{
    takesString($i);
}
===expect===
ImplicitToStringCast
===ignore===
TODO
