===description===
catch type cross file missing
===file:Exceptions.php===
<?php
namespace App;
class RealException extends \Exception {}
===file:Handler.php===
<?php
use App\RealException;
use App\MissingException;
function handle(): void {
    try {
        throw new \Exception();
    } catch (RealException $e) {
    } catch (MissingException $e) {
    }
}
===expect===
Handler.php: MissingThrowsDocblock@6:9: Exception Exception is thrown but not declared in @throws
Handler.php: UnusedVariable@7:13: Variable $e is never read
Handler.php: UndefinedClass@8:14: Class App\MissingException does not exist
