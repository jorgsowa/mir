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
Handler.php: UndefinedClass: Class App\MissingException does not exist
Handler.php: UnusedVariable: Variable $e is never read
Handler.php: UnusedVariable: Variable $e is never read
