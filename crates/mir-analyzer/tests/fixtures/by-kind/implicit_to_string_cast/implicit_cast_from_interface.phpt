===description===
No ImplicitToStringCast when passing a type that declares __toString via interface — __toString is sufficient
===config===
suppress=MissingReturnType,UnusedParam
===file===
<?php
interface I {
    public function __toString();
}

/** @mutation-free */
function takesString(string $str): void { }

function takesI(I $i): void
{
    takesString($i);
}
===expect===
