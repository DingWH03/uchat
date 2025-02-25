import { useState } from 'react';
import "../styles/pages/MainPage.css";

function MainPage() {
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [serverIp, setServerIp] = useState('');
    const [serverPort, setServerPort] = useState('');

    const openModal = () => setIsModalOpen(true);
    const closeModal = () => setIsModalOpen(false);

    const handleIpChange = (e: React.ChangeEvent<HTMLInputElement>) => setServerIp(e.target.value);
    const handlePortChange = (e: React.ChangeEvent<HTMLInputElement>) => setServerPort(e.target.value);

    return (
        <main className="container">
            <h1>Welcome to uchat</h1>

            <div>
                <button type="button" onClick={() => window.location.href = '/login'}>Login</button>
                <button type="button" onClick={() => window.location.href = '/signup'}>Sign up</button>
            </div>

            {/* 按钮放在左下角 */}
            <button
                type="button"
                onClick={openModal}
                className="server-info-btn"
            >
                Server Info
            </button>

            {isModalOpen && (
                <div className="modal">
                    <div className="modal-content">
                        <h2>Server Information</h2>
                        <label>
                            Server IP:
                            <input
                                type="text"
                                placeholder="Server IP"
                                value={serverIp}
                                onChange={handleIpChange}
                            />
                        </label>
                        <br />
                        <label>
                            Server Port:
                            <input
                                type="text"
                                placeholder="Server Port"
                                value={serverPort}
                                onChange={handlePortChange}
                            />
                        </label>
                        <br />
                        <button type="button" onClick={closeModal}>Close</button>
                    </div>
                </div>
            )}
        </main>
    );
}

export default MainPage;
