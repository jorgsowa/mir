===description===
An imported async-client interface must resolve in a class implements list.
===file===
<?php
namespace Http\Client;
interface HttpAsyncClient {}

namespace App;
use Http\Client\HttpAsyncClient;

final class Client implements HttpAsyncClient {}
===expect===
