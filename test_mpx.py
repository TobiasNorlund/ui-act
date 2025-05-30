import pytest
from unittest.mock import patch, MagicMock
import subprocess
from mpx import get_screen_resolution, main, e

def test_get_screen_resolution_success():
    # Mock xdpyinfo output with a known resolution
    mock_output = "dimensions:    1920x1080"
    with patch('subprocess.run') as mock_run:
        mock_process = MagicMock()
        mock_process.stdout = mock_output
        mock_run.return_value = mock_process
        
        width, height = get_screen_resolution()
        assert width == 1920
        assert height == 1080

def test_get_screen_resolution_failure():
    # Mock xdpyinfo output with invalid format
    mock_output = "invalid output"
    with patch('subprocess.run') as mock_run:
        mock_process = MagicMock()
        mock_process.stdout = mock_output
        mock_run.return_value = mock_process
        
        with pytest.raises(RuntimeError) as exc_info:
            get_screen_resolution()
        assert "Could not detect screen resolution" in str(exc_info.value)

def test_get_screen_resolution_command_error():
    # Mock subprocess.run raising an exception
    with patch('subprocess.run') as mock_run:
        mock_run.side_effect = subprocess.SubprocessError("Command failed")
        
        with pytest.raises(RuntimeError) as exc_info:
            get_screen_resolution()
        assert "Could not detect screen resolution" in str(exc_info.value)

@patch('mpx.get_screen_resolution')
@patch('mpx.UInput')
@patch('mpx.get_device_id')
@patch('subprocess.run')
def test_main_success(mock_run, mock_get_device_id, mock_uinput, mock_get_resolution):
    # Mock screen resolution
    mock_get_resolution.return_value = (1920, 1080)
    
    # Mock UInput instance
    mock_ui = MagicMock()
    mock_uinput.return_value = mock_ui
    
    # Mock device IDs
    mock_get_device_id.side_effect = [1, 2]  # First call for mouse, second for master
    
    # Mock subprocess.run calls
    mock_process = MagicMock()
    mock_process.returncode = 0
    mock_run.return_value = mock_process
    
    # Run main function
    main()
    
    # Verify UInput was created with correct capabilities
    mock_uinput.assert_called_once()
    capabilities = mock_uinput.call_args[0][0]
    
    # Check X axis capabilities
    assert capabilities[e.EV_ABS][0][0] == e.ABS_X
    assert capabilities[e.EV_ABS][0][1].max == 1919  # width - 1
    
    # Check Y axis capabilities
    assert capabilities[e.EV_ABS][1][0] == e.ABS_Y
    assert capabilities[e.EV_ABS][1][1].max == 1079  # height - 1
    
    # Check button capabilities
    assert e.BTN_LEFT in capabilities[e.EV_KEY]
    
    # Verify device was closed
    mock_ui.close.assert_called_once()

@patch('mpx.get_screen_resolution')
@patch('mpx.UInput')
@patch('mpx.get_device_id')
def test_main_keyboard_interrupt(mock_get_device_id, mock_uinput, mock_get_resolution):
    # Mock screen resolution
    mock_get_resolution.return_value = (1920, 1080)
    
    # Mock UInput instance
    mock_ui = MagicMock()
    mock_uinput.return_value = mock_ui
    
    # Mock device ID
    mock_get_device_id.return_value = 1
    
    # Mock write to raise KeyboardInterrupt
    mock_ui.write.side_effect = KeyboardInterrupt()
    
    # Run main function
    main()
    
    # Verify device was closed even after interruption
    mock_ui.close.assert_called_once()

@patch('mpx.get_screen_resolution')
@patch('mpx.UInput')
def test_main_device_error(mock_uinput, mock_get_resolution):
    # Mock screen resolution
    mock_get_resolution.return_value = (1920, 1080)
    
    # Mock UInput to raise an error
    mock_uinput.side_effect = Exception("Device creation failed")
    
    # Run main function and verify it handles the error gracefully
    with pytest.raises(Exception) as exc_info:
        main()
    assert "Device creation failed" in str(exc_info.value) 