Option Explicit

Dim shell, fso, folder, exitCode, logPath
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")

folder = fso.GetParentFolderName(WScript.ScriptFullName)
shell.CurrentDirectory = folder
logPath = folder & "\orange-server-stop.log"

' Stop the Orange Compose services silently. The database volume is preserved.
exitCode = shell.Run("cmd.exe /c docker compose stop > orange-server-stop.log 2>&1", 0, True)

If exitCode = 0 Then
    MsgBox "Orange Server wurde gestoppt. Deine Datenbank bleibt erhalten.", vbInformation, "Orange Server"
Else
    MsgBox "Orange Server konnte nicht gestoppt werden." & vbCrLf & _
        "Fehlerdetails: " & logPath, vbExclamation, "Orange Server"
End If
