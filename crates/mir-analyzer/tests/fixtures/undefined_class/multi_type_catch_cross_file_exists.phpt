===description===
multi-type catch cross file exists — no UndefinedClass for any catch type when both are defined in other files
===file:Exceptions.php===
<?php
namespace App;
class NetworkError extends \Exception {}
class TimeoutError extends \Exception {}
===file:Client.php===
<?php
use App\NetworkError;
use App\TimeoutError;
/**
 * @throws \Exception
 */
function fetch(): void {
    try {
        throw new \Exception();
    } catch (NetworkError | TimeoutError $e) {
        echo $e->getMessage();
    }
}
===expect===
