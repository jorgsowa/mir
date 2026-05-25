===description===
catch type cross file missing
===config===
suppress=MissingThrowsDocblock,UnusedVariable,UnusedFunction
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
Handler.php: UndefinedClass@8:14: Class App\MissingException does not exist
