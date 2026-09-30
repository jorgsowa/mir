===description===
Forbidden throwable implementation
===file===
<?php
class C implements Throwable {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::getMessage() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::getCode() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::getFile() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::getLine() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::getTrace() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::getTraceAsString() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::getPrevious() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Throwable::__toString() from interface
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class C must implement Stringable::__toString() from interface

===expect===
