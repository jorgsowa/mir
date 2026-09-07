===description===
A static call through a variable receiver typed as a plain interface
($p::method()) dispatches at runtime to whichever concrete class $p actually
holds — valid PHP. Only a literal Interface::method() receiver is genuinely
invalid (see static_interface_call.phpt).
===config===
suppress=UnusedParam
===file===
<?php

interface Provider {
    public static function getDefinitions(): array;
}

function run(Provider $p): void {
    $p::getDefinitions();
}
===expect===
