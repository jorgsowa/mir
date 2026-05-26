===description===
catch type cross file exists — no UndefinedClass when exception class is defined in another file
===file:Exceptions.php===
<?php
namespace App;
class AppException extends \Exception {}
===file:Handler.php===
<?php
use App\AppException;
/**
 * @throws \Exception
 */
function handle(): void {
    try {
        throw new \Exception();
    } catch (AppException $e) {
        echo $e->getMessage();
    }
}
===expect===
