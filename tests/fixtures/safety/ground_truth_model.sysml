package AutonomousVehicle_SSOT {
    doc /* SSOT for Autonomous System Architecture and Safety Model */

    attribute actuatorCount : Integer = 4;
    attribute topologyConfiguration : String = "Cruciform";
    attribute catapultLaunchLimitG : Real = 12.0;
    attribute maxGLoad : Real = 12.0;

    part def Airframe {
        attribute massKg : Real = 25.0;
    }

    part def FlightControlComputer {
        port c2_bus : RS485;
        port telemetry : MAVLink;
    }
}
