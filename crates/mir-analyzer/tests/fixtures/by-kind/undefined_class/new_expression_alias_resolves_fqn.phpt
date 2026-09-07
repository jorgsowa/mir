===description===
new ClassName() via use-import alias resolves the fully-qualified name — no
UndefinedClass is emitted and the result type carries the FQN, enabling
correct hover and go-to-definition behaviour.
===config===
suppress=UnusedVariable,UnusedFunction
===file:RequestGuard.php===
<?php
namespace Illuminate\Auth;
class RequestGuard {}
===file:AuthManager.php===
<?php
use Illuminate\Auth\RequestGuard;
function make(): void {
    $g = new RequestGuard();
    /** @mir-check $g is Illuminate\Auth\RequestGuard */
}
===expect===
