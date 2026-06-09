===description===
Forbidden throwable implementation
===file===
<?php
class C implements Throwable {}

===expect===
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::getMessage() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::getCode() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::getFile() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::getLine() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::getTrace() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::getTraceAsString() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::getPrevious() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Throwable::__toString() from interface
UnimplementedInterfaceMethod@2:0-2:31: Class C must implement Stringable::__toString() from interface
