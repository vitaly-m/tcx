# TCX

The rust library to interact with XML files in TCX format. The implementation is based on
[TrainingCenterDatabasev2](https://www8.garmin.com/xmlschemas/TrainingCenterDatabasev2.xsd) and
[ActivityExtensionv2](https://www8.garmin.com/xmlschemas/ActivityExtensionv2.xsd) schemas.

## Supported High Level Types

Types mentioned below are supported with all required subtypes for them:

* ActivityList_t and all required types
* AbstractSource_t
    * Device_t
    * Application_t
* Extensions:
    * ActivityTrackpointExtension_t
    * ActivityLapExtension_t