===description===
Method-level @psalm-type overrides a class-level alias with the same name without false positives
===config===
suppress=UnusedParam
===file===
<?php
namespace App;

/**
 * @psalm-type Token = string
 */
class Auth {
    /**
     * @psalm-type Token = array{value: string, expiry: int}
     * @param Token $token
     * @return bool
     */
    public function validate(array $token): bool {
        return $token['expiry'] > 0;
    }
}

$auth = new Auth();
$auth->validate(['value' => 'abc', 'expiry' => 9999]);
===expect===
