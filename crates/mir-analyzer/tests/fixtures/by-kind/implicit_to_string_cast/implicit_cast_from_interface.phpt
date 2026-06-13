===description===
Implicit cast from interface
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
ImplicitToStringCast@11:17-11:19: Class I is implicitly cast to string
