===description===
An attributed parameter is reported at its declaration, not at the attribute line.
===file===
<?php
function login(
    #[\SensitiveParameter]
    string $secret,
//  ^^^^^^^^^^^^^^ UnusedParam: Parameter $secret is never used
): void {}
