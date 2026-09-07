===description===
new LocalAlias() where the import is renamed (use X\Y as Z) resolves to the
original FQN — the ClassReference carries X\Y, not the local alias Z.
===config===
suppress=UnusedVariable,UnusedFunction
===file:RequestGuard.php===
<?php
namespace Illuminate\Auth;
class RequestGuard {}
===file:AuthManager.php===
<?php
use Illuminate\Auth\RequestGuard as Guard;
function make(): void {
    $g = new Guard();
    /** @mir-check $g is Illuminate\Auth\RequestGuard */
}
===expect===
