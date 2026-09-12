===description===
Imported interfaces resolve in implements clauses.
===file===
<?php
namespace Http\Client;
interface HttpAsyncClient {}

namespace App;
use Http\Client\HttpAsyncClient;

final class Client implements HttpAsyncClient {}
===expect===
