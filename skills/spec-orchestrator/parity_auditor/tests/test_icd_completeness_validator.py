"""
Test module for ICD Completeness Validator.
/// Realises: [Issue310/ICDCompletenessValidator]
"""
import os
import pytest
from unittest.mock import patch, mock_open, MagicMock

from parity_auditor.validators.icd_completeness_validator import ICDCompletenessValidator
from parity_auditor.core.workspace import WorkspaceRepository

class TestICDCompletenessValidator:
    """
    Test suite for ICD completeness validation checks.
    /// Realises: [Issue310/ICDCompletenessValidator]
    """

    @patch("parity_auditor.validators.icd_completeness_validator._find_sysml_files")
    @patch("parity_auditor.validators.icd_completeness_validator.os.path.exists")
    @patch("builtins.open")
    def test_sysml_port_missing_from_roster(self, mock_file, mock_exists, mock_find_files):
        """
        Tests that an error is raised if a SysML port is missing from ICD_01 roster.
        /// Realises: [Issue310/ICDCompletenessValidator]
        """
        mock_find_files.return_value = ["fake.sysml"]
        mock_exists.return_value = True

        sysml_content = "part def SubsystemA { out port DataOut: float32; }"
        icd01_content = " | Port ID | Subsystem | Port Name | Direction |\\n |---|---|---|---|\\n "
        icd02_content = ""

        def side_effect(filename, *args, **kwargs):
            if "fake.sysml" in filename:
                return mock_open(read_data=sysml_content).return_value
            elif "ICD_01" in filename:
                return mock_open(read_data=icd01_content).return_value
            elif "ICD_02" in filename:
                return mock_open(read_data=icd02_content).return_value
            return mock_open(read_data="").return_value

        mock_file.side_effect = side_effect

        validator = ICDCompletenessValidator()
        repo = MagicMock(spec=WorkspaceRepository)
        repo.workspace_dir = "/fake/workspace"
        
        findings = validator.validate(repo)
        
        missing_port_findings = [f for f in findings if f.rule_id == "icd-port-missing-from-roster"]
        assert len(missing_port_findings) == 1
        assert "DataOut" in missing_port_findings[0]

    @patch("parity_auditor.validators.icd_completeness_validator._find_sysml_files")
    @patch("parity_auditor.validators.icd_completeness_validator.os.path.exists")
    @patch("builtins.open")
    def test_sysml_connection_missing_from_roster(self, mock_file, mock_exists, mock_find_files):
        """
        Tests that an error is raised if a SysML connection is missing from ICD_01 roster.
        /// Realises: [Issue310/ICDCompletenessValidator]
        """
        mock_find_files.return_value = ["fake.sysml"]
        mock_exists.return_value = True

        sysml_content = "connect SubsystemA.DataOut to SubsystemB.DataIn"
        icd01_content = " | Connection ID | Source Port | Dest Port |\\n |---|---|---|\\n "
        icd02_content = ""

        def side_effect(filename, *args, **kwargs):
            if "fake.sysml" in filename:
                return mock_open(read_data=sysml_content).return_value
            elif "ICD_01" in filename:
                return mock_open(read_data=icd01_content).return_value
            elif "ICD_02" in filename:
                return mock_open(read_data=icd02_content).return_value
            return mock_open(read_data="").return_value

        mock_file.side_effect = side_effect

        validator = ICDCompletenessValidator()
        repo = MagicMock(spec=WorkspaceRepository)
        repo.workspace_dir = "/fake/workspace"
        
        findings = validator.validate(repo)
        
        missing_conn_findings = [f for f in findings if f.rule_id == "icd-connection-missing-from-roster"]
        assert len(missing_conn_findings) == 1
        assert "SubsystemA.DataOut" in missing_conn_findings[0]
