Option Explicit

Dim shell, fso, folder, exitCode, logPath
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")

folder = fso.GetParentFolderName(WScript.ScriptFullName)
shell.CurrentDirectory = folder
logPath = folder & "\orange-server-start.log"

' Run Docker Compose silently and save output for troubleshooting.
exitCode = shell.Run("cmd.exe /c docker compose up --build -d > orange-server-start.log 2>&1", 0, True)

If exitCode = 0 Then
    MsgBox "Orange Server wurde gestartet." & vbCrLf & vbCrLf & _
        "Prüfe den Status hier:" & vbCrLf & _
        "http://localhost:8080/health" & vbCrLf & vbCrLf & _
        "Details: " & logPath, vbInformation, "Orange Server"
Else
    MsgBox "Orange Server konnte nicht gestartet werden." & vbCrLf & vbCrLf & _
        "Öffne Docker Desktop und versuche es erneut." & vbCrLf & _
        "Fehlerdetails: " & logPath, vbExclamation, "Orange Server"
End If
